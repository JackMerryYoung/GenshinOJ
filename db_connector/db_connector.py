import pymysql

class db_connector:
    def __init__(self, server_instance):
        self.DatabaseConnection()
    
    def DatabaseConnection(self, DATABASE_USER = 'root', DATABASE_PASSWORD = '123456', DATABASE_PORT = 3306): # Database Connection
        try: # Connect to database
            self.database = pymysql.connect(host = 'localhost', user = DATABASE_USER, passwd = DATABASE_PASSWORD, port = DATABASE_PORT, database = 'GenshinOJ')
            print('连接成功！')
        except Exception: # If the database is not established, create it
            self.database = pymysql.connect(host = 'localhost', user = DATABASE_USER, passwd = DATABASE_PASSWORD, port = DATABASE_PORT)
            self.database_cursor = self.database.cursor(); self.database_cursor.execute('CREATE DATABASE IF NOT EXISTS GenshinOJ;')
            self.database = pymysql.connect(host = 'localhost', user = DATABASE_USER, passwd = DATABASE_PASSWORD, port = DATABASE_PORT, database = 'GenshinOJ')
            print('初始化完毕！')

        # Get MySQL's version
        self.database_cursor = self.database.cursor(); self.database_cursor.execute('SELECT VERSION()')
        print(f'MySQL Version: {self.database_cursor.fetchone()[0]}')
        # Initialize the table
        self.database_cursor.execute(
        '''
        CREATE TABLE IF NOT EXISTS users (
            id INT AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(256) NOT NULL,
            password VARCHAR(256) NOT NULL
        )
        '''
        )